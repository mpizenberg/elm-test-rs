port module ElmTestRs.Test.Runner exposing (Program, start)

import Array
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Internal.SeededRunners as SeededRunners exposing (SeededRunners)
import Json.Encode exposing (Value)
import Platform
import Test exposing (Test)



-- Ports


port askNbTests : (Value -> msg) -> Sub msg


port sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg


port receiveRunTest : (Int -> msg) -> Sub msg


port sendResult : Value -> Cmd msg



-- Types


type alias Program =
    Platform.Program Flags Model Msg


type alias Flags =
    { initialSeed : Int
    , fuzzRuns : Int
    }


type alias Model =
    { testRunners : SeededRunners }


type Msg {- ReceiveRunTest: order from the supervisor via port -}
    = AskNbTests
    | ReceiveRunTest Int



-- Functions


start : Test -> Program
start masterTest =
    Platform.worker
        { init = init masterTest
        , update = update
        , subscriptions = \_ -> Sub.batch [ askNbTests (always AskNbTests), receiveRunTest ReceiveRunTest ]
        }


init : Test -> Flags -> ( Model, Cmd Msg )
init masterTest flags =
    ( Model (SeededRunners.fromTest masterTest flags), Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.testRunners ) of
        -- AskNbTests
        ( AskNbTests, Ok { runners } ) ->
            ( model, sendTypedNbTests (Array.length runners) )

        ( AskNbTests, Err _ ) ->
            ( model, Debug.todo "Deal with invalid runners" )

        -- ReceiveRunTest
        ( ReceiveRunTest id, Ok { runners } ) ->
            ( model, sendTestResult id (SeededRunners.run id runners) )

        ( ReceiveRunTest _, Err _ ) ->
            ( model, Debug.todo "Deal with invalid runners" )


sendTypedNbTests : Int -> Cmd msg
sendTypedNbTests nbTests =
    sendNbTests { type_ = "nbTests", nbTests = nbTests }


sendTestResult : Int -> Maybe TestResult -> Cmd msg
sendTestResult id maybeResult =
    case maybeResult of
        Nothing ->
            Cmd.none

        Just result ->
            sendResult <|
                Json.Encode.object
                    [ ( "type_", Json.Encode.string "result" )
                    , ( "id", Json.Encode.int id )
                    , ( "result", TestResult.encode result )
                    ]

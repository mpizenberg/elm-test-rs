module ElmTestRs.Test.Runner exposing (Msg, Ports, Program, worker)

import Array
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Internal.SeededRunners as SeededRunners exposing (SeededRunners)
import Json.Encode exposing (Value)
import Platform
import Test exposing (Test)



-- Ports


type alias Ports msg =
    { askNbTests : (Value -> msg) -> Sub msg
    , sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg
    , receiveRunTest : (Int -> msg) -> Sub msg
    , sendResult : Value -> Cmd msg
    }



-- Types


type alias Program msg =
    Platform.Program Flags (Model msg) Msg


type alias Flags =
    { initialSeed : Int
    , fuzzRuns : Int
    }


type alias Model msg =
    { ports : Ports msg
    , testRunners : SeededRunners
    }


type Msg {- ReceiveRunTest: order from the supervisor via port -}
    = AskNbTests
    | ReceiveRunTest Int



-- Functions


worker : Ports Msg -> Test -> Program Msg
worker ({ askNbTests, receiveRunTest } as ports) masterTest =
    Platform.worker
        { init = init masterTest ports
        , update = update
        , subscriptions = \_ -> Sub.batch [ askNbTests (always AskNbTests), receiveRunTest ReceiveRunTest ]
        }


init : Test -> Ports Msg -> Flags -> ( Model Msg, Cmd Msg )
init masterTest ports flags =
    ( Model ports (SeededRunners.fromTest masterTest flags), Cmd.none )


update : Msg -> Model Msg -> ( Model Msg, Cmd Msg )
update msg model =
    case ( msg, model.testRunners ) of
        -- AskNbTests
        ( AskNbTests, Ok { runners } ) ->
            ( model, sendTypedNbTests model.ports (Array.length runners) )

        ( AskNbTests, Err _ ) ->
            ( model, Debug.todo "Deal with invalid runners" )

        -- ReceiveRunTest
        ( ReceiveRunTest id, Ok { runners } ) ->
            ( model, sendTestResult model.ports id (SeededRunners.run id runners) )

        ( ReceiveRunTest _, Err _ ) ->
            ( model, Debug.todo "Deal with invalid runners" )


sendTypedNbTests : Ports msg -> Int -> Cmd msg
sendTypedNbTests ports nbTests =
    ports.sendNbTests { type_ = "nbTests", nbTests = nbTests }


sendTestResult : Ports msg -> Int -> Maybe TestResult -> Cmd msg
sendTestResult ports id maybeResult =
    case maybeResult of
        Nothing ->
            Cmd.none

        Just result ->
            ports.sendResult <|
                Json.Encode.object
                    [ ( "type_", Json.Encode.string "result" )
                    , ( "id", Json.Encode.int id )
                    , ( "result", TestResult.encode result )
                    ]

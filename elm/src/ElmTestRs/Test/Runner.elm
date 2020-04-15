port module ElmTestRs.Test.Runner exposing (Program, start)

import Array exposing (Array)
import ElmTestRs.Test.Result
import Json.Encode exposing (Value)
import Platform
import Random
import Test exposing (Test)
import Test.Runner exposing (Runner)



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
    { testRunners : Runners }


type Runners
    = Plain (Array Runner)
    | Only (Array Runner)
    | Skipping (Array Runner)
    | Invalid String


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
init masterTest { initialSeed, fuzzRuns } =
    let
        seededRunners =
            Test.Runner.fromTest fuzzRuns (Random.initialSeed initialSeed) masterTest

        testRunners =
            case seededRunners of
                Test.Runner.Plain runnerList ->
                    Plain (Array.fromList runnerList)

                Test.Runner.Only runnerList ->
                    Only (Array.fromList runnerList)

                Test.Runner.Skipping runnerList ->
                    Skipping (Array.fromList runnerList)

                Test.Runner.Invalid error ->
                    Invalid error
    in
    ( Model testRunners, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.testRunners ) of
        -- AskNbTests
        ( AskNbTests, Plain runners ) ->
            ( model, sendTypedNbTests (Array.length runners) )

        ( AskNbTests, Only runners ) ->
            ( model, sendTypedNbTests (Array.length runners) )

        ( AskNbTests, Skipping runners ) ->
            ( model, sendTypedNbTests (Array.length runners) )

        ( AskNbTests, Invalid error ) ->
            ( model, sendTypedNbTests 1 )

        -- ReceiveRunTest
        ( ReceiveRunTest id, Plain runners ) ->
            ( model, runPlain id runners )

        ( ReceiveRunTest id, Only runners ) ->
            ( model, runOnly id runners )

        ( ReceiveRunTest id, Skipping runners ) ->
            ( model, runSkipping id runners )

        ( ReceiveRunTest id, Invalid error ) ->
            ( model, runInvalid id error )


sendTypedNbTests : Int -> Cmd msg
sendTypedNbTests nbTests =
    sendNbTests { type_ = "nbTests", nbTests = nbTests }


runPlain : Int -> Array Runner -> Cmd Msg
runPlain id runners =
    case Array.get id runners of
        Nothing ->
            Cmd.none

        Just runner ->
            runner.run ()
                |> ElmTestRs.Test.Result.fromExpectations runner.labels
                |> ElmTestRs.Test.Result.encode
                |> sendResultWithId id


sendResultWithId : Int -> Value -> Cmd msg
sendResultWithId id result =
    sendResult <|
        Json.Encode.object
            [ ( "type_", Json.Encode.string "result" )
            , ( "id", Json.Encode.int id )
            , ( "result", result )
            ]



-- TODO: figure out what happens when there is an Only or Skipping case


runOnly : Int -> Array Runner -> Cmd Msg
runOnly id runners =
    Debug.todo ("runOnly " ++ String.fromInt id)


runSkipping : Int -> Array Runner -> Cmd Msg
runSkipping id runners =
    Debug.todo ("runSkipping " ++ String.fromInt id)


runInvalid : Int -> String -> Cmd Msg
runInvalid id error =
    Debug.todo ("runInvalid " ++ String.fromInt id)

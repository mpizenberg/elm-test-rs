port module ElmTestRs.Test.Runner exposing (Program, start)

import Array exposing (Array)
import ElmTestRs.Test.Result exposing (TestResult)
import Json.Encode as Encode exposing (Value)
import Platform
import Random
import Test exposing (Test)
import Test.Runner exposing (Runner)



-- Ports


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
    = ReceiveRunTest Int



-- Functions


start : Test -> Program
start masterTest =
    Platform.worker
        { init = init masterTest
        , update = update
        , subscriptions = \_ -> receiveRunTest ReceiveRunTest
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
        ( ReceiveRunTest id, Plain runners ) ->
            ( model, runPlain id runners )

        ( ReceiveRunTest id, Only runners ) ->
            ( model, runOnly id runners )

        ( ReceiveRunTest id, Skipping runners ) ->
            ( model, runSkipping id runners )

        ( ReceiveRunTest id, Invalid error ) ->
            ( model, runInvalid id error )


runPlain : Int -> Array Runner -> Cmd Msg
runPlain id runners =
    case Array.get id runners of
        Nothing ->
            Cmd.none

        Just runner ->
            runner.run ()
                |> ElmTestRs.Test.Result.fromExpectations runner.labels
                |> ElmTestRs.Test.Result.encode
                |> sendResult



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

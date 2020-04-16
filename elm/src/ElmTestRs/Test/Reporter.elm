module ElmTestRs.Test.Reporter exposing (Flags, Model, Msg, Ports, worker)

import Array exposing (Array)
import ElmTestRs.Test.Reporter.Console as ReporterConsole
import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Reporter.Json as ReporterJson
import ElmTestRs.Test.Reporter.Junit as ReporterJunit
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Json.Decode exposing (Value, decodeValue)
import Task


type alias Ports msg =
    { restart : (Int -> msg) -> Sub msg
    , incomingResult : (Value -> msg) -> Sub msg
    , stdout : String -> Cmd msg
    , signalFinished : Int -> Cmd msg
    }


worker : Ports Msg -> Program Flags Model Msg
worker ({ restart, incomingResult } as ports) =
    Platform.worker
        { init = init ports
        , update = update
        , subscriptions = \_ -> Sub.batch [ restart Restart, incomingResult IncomingResult ]
        }



-- Types


type alias Flags =
    { initialSeed : Int
    , fuzzRuns : Int
    , mode : String
    }


type alias Model =
    { ports : Ports Msg
    , reporter : Interface
    , nbTests : Int
    , testResults : Array TestResult
    }


type Msg
    = Restart Int
    | IncomingResult Value
    | Summarize
    | Finished



-- Functions


chooseReporter : Flags -> Interface
chooseReporter { initialSeed, fuzzRuns, mode } =
    case mode of
        "json" ->
            ReporterJson.implementation { seed = initialSeed, fuzzRuns = fuzzRuns }

        "junit" ->
            ReporterJunit.implementation

        _ ->
            ReporterConsole.implementation { seed = initialSeed, fuzzRuns = fuzzRuns }


init : Ports Msg -> Flags -> ( Model, Cmd Msg )
init ports flags =
    let
        reporter =
            chooseReporter flags
    in
    ( Model ports reporter 0 Array.empty, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Restart nbTests ->
            ( Model model.ports model.reporter nbTests Array.empty
            , report model.ports.stdout (model.reporter.onBegin nbTests)
            )

        IncomingResult value ->
            let
                testResultResult =
                    decodeValue TestResult.decoder value

                allTestResults =
                    case testResultResult of
                        Ok testResult ->
                            Array.push testResult model.testResults

                        Err _ ->
                            model.testResults

                updatedModel =
                    { model | testResults = allTestResults }
            in
            if Array.length updatedModel.testResults == model.nbTests then
                ( updatedModel
                , Result.map model.reporter.onResult testResultResult
                    |> Result.map (reportAndThenSummarize model.ports.stdout)
                    |> Result.withDefault Cmd.none
                )

            else
                ( updatedModel
                , Result.map model.reporter.onResult testResultResult
                    |> Result.map (report model.ports.stdout)
                    |> Result.withDefault Cmd.none
                )

        Summarize ->
            ( model, summarize model.ports.stdout (model.reporter.onEnd model.testResults) )

        Finished ->
            ( model, model.ports.signalFinished (errorCode model.testResults) )


errorCode : Array TestResult -> Int
errorCode testResults =
    let
        { nbFailed } =
            TestResult.summary testResults
    in
    if nbFailed > 0 then
        2

    else
        0


reportAndThenSummarize : (String -> Cmd Msg) -> Maybe String -> Cmd Msg
reportAndThenSummarize stdout content =
    Cmd.batch [ report stdout content, commandMsg Summarize ]


report : (String -> Cmd Msg) -> Maybe String -> Cmd Msg
report stdout content =
    Maybe.map stdout content
        |> Maybe.withDefault Cmd.none


summarize : (String -> Cmd Msg) -> Maybe String -> Cmd Msg
summarize stdout content =
    case content of
        Just string ->
            Cmd.batch [ stdout string, commandMsg Finished ]

        Nothing ->
            commandMsg Finished


commandMsg : msg -> Cmd msg
commandMsg msg =
    Task.perform identity (Task.succeed msg)

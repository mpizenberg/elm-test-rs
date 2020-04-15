port module ElmTestRs.Test.Reporter exposing (main)

import Array exposing (Array)
import ElmTestRs.Test.Reporter.Console as ReporterConsole
import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Reporter.Json as ReporterJson
import ElmTestRs.Test.Reporter.Junit as ReporterJunit
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Json.Decode exposing (Value, decodeValue)
import Task


port restart : (Int -> msg) -> Sub msg


port incomingResult : (Value -> msg) -> Sub msg


port signalFinished : Int -> Cmd msg


port stdout : String -> Cmd msg


main : Program Flags Model Msg
main =
    Platform.worker
        { init = init
        , update = update
        , subscriptions = subscriptions
        }



-- Types


type alias Flags =
    { initialSeed : Int
    , fuzzRuns : Int
    , mode : String
    }


type alias Model =
    { reporter : Interface
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


init : Flags -> ( Model, Cmd Msg )
init flags =
    let
        reporter =
            chooseReporter flags
    in
    ( Model reporter 0 Array.empty, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Restart nbTests ->
            ( Model model.reporter nbTests Array.empty, report model.reporter.onBegin nbTests )

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
                , Result.withDefault Cmd.none (Result.map (reportAndThenSummarize model.reporter.onResult) testResultResult)
                )

            else
                ( updatedModel
                , Result.withDefault Cmd.none (Result.map (report model.reporter.onResult) testResultResult)
                )

        Summarize ->
            ( model, summarize model.reporter.onEnd model.testResults )

        Finished ->
            ( model, signalFinished (errorCode model.testResults) )


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


reportAndThenSummarize : (TestResult -> Maybe String) -> TestResult -> Cmd Msg
reportAndThenSummarize onResult testResult =
    Cmd.batch [ report onResult testResult, commandMsg Summarize ]


report : (a -> Maybe String) -> a -> Cmd Msg
report writer data =
    case writer data of
        Just string ->
            stdout string

        Nothing ->
            Cmd.none


summarize : (Array TestResult -> Maybe String) -> Array TestResult -> Cmd Msg
summarize onEnd testResults =
    case onEnd testResults of
        Just string ->
            Cmd.batch [ stdout string, commandMsg Finished ]

        Nothing ->
            commandMsg Finished


commandMsg : msg -> Cmd msg
commandMsg msg =
    Task.perform identity (Task.succeed msg)


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ restart Restart
        , incomingResult IncomingResult
        ]

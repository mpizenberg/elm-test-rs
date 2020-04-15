port module ElmTestRs.Test.Reporter exposing (main)

import Array exposing (Array)
import ElmTestRs.Test.Reporter.Console
import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Reporter.Json
import ElmTestRs.Test.Reporter.Junit
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Json.Decode exposing (Value, decodeValue)
import Task


port restart : (Int -> msg) -> Sub msg


port incomingResult : (Value -> msg) -> Sub msg


port signalFinished : String -> Cmd msg


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


chooseReporter : String -> Interface
chooseReporter str =
    case str of
        "json" ->
            ElmTestRs.Test.Reporter.Json.implementation

        "junit" ->
            ElmTestRs.Test.Reporter.Junit.implementation

        _ ->
            ElmTestRs.Test.Reporter.Console.implementation


init : Flags -> ( Model, Cmd Msg )
init { initialSeed, fuzzRuns, mode } =
    let
        reporter =
            chooseReporter mode
    in
    ( Model reporter 0 Array.empty, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Restart nbTests ->
            ( Model model.reporter nbTests Array.empty, report model.reporter.onBegin () )

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
            ( model, signalFinished "Finished!" )


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

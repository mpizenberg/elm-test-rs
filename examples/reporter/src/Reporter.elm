port module Reporter exposing (main)

import Array exposing (Array)
import Data
import Json.Decode exposing (decodeValue)
import Json.Encode as Encode
import Reporter.Console
import Reporter.Interface exposing (Interface)
import Reporter.Json
import Reporter.Junit
import Task


port incomingResult : (Encode.Value -> msg) -> Sub msg


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
    { mode : String
    , nbTests : Int
    }


type alias Model =
    { reporter : Interface
    , nbTests : Int
    , testResults : Array Data.TestResult
    }


type Msg
    = IncomingResult Encode.Value
    | Summarize
    | Finished



-- Functions


chooseReporter : String -> Interface
chooseReporter str =
    case str of
        "json" ->
            Reporter.Json.implementation

        "junit" ->
            Reporter.Junit.implementation

        _ ->
            Reporter.Console.implementation


init : Flags -> ( Model, Cmd Msg )
init { mode, nbTests } =
    let
        reporter =
            chooseReporter mode
    in
    ( Model reporter nbTests Array.empty, report reporter.onBegin () )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        IncomingResult value ->
            let
                testResultResult =
                    decodeValue Data.testResultDecoder value

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


reportAndThenSummarize : (Data.TestResult -> Maybe String) -> Data.TestResult -> Cmd Msg
reportAndThenSummarize onResult testResult =
    Cmd.batch [ report onResult testResult, commandMsg Summarize ]


report : (a -> Maybe String) -> a -> Cmd Msg
report writer data =
    case writer data of
        Just string ->
            stdout string

        Nothing ->
            Cmd.none


summarize : (Array Data.TestResult -> Maybe String) -> Array Data.TestResult -> Cmd Msg
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
    incomingResult IncomingResult

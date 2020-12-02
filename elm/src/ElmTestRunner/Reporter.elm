module ElmTestRunner.Reporter exposing
    ( worker, Ports
    , Flags, Model, Msg
    )

{-| Main module for a test reporter.


# Create a main test reporter

@docs worker, Ports


# Internal types for function signatures

@docs Flags, Model, Msg

-}

import Array exposing (Array)
import ElmTestRunner.Reporter.Console as ReporterConsole
import ElmTestRunner.Reporter.Interface exposing (Interface)
import ElmTestRunner.Reporter.Json as ReporterJson
import ElmTestRunner.Reporter.Junit as ReporterJunit
import ElmTestRunner.Result as TestResult exposing (TestResult)
import Json.Decode exposing (Value, decodeValue)
import Task


{-| Ports(ish) required by the worker program to function.
They aren't necessarily exactly ports
but will basically be wrapped by an actual port in the main Elm caller module.
-}
type alias Ports msg =
    { restart : (Int -> msg) -> Sub msg
    , incomingResult : (Value -> msg) -> Sub msg
    , stdout : String -> Cmd msg
    , signalFinished : Int -> Cmd msg
    }


{-| Create a tests reporter.
Some specific ports(ish) are required as arguments,
The main Elm module calling this one will typically look like the example below.

    port module Reporter exposing (main)

    import ElmTestRunner.Reporter exposing (Flags, Model, Msg)
    import Json.Decode exposing (Value)

    port restart : (Int -> msg) -> Sub msg

    port incomingResult : (Value -> msg) -> Sub msg

    port signalFinished : Int -> Cmd msg

    port stdout : String -> Cmd msg

    main : Program Flags Model Msg
    main =
        ElmTestRunner.Reporter.worker
            { restart = restart
            , incomingResult = incomingResult
            , stdout = stdout
            , signalFinished = signalFinished
            }

It can later be called with a tiny bit of JS similar to:

```js
// Start the Elm app
const { Elm } = require("./Reporter.elm.js");
const flags = {
  initialSeed: {{ initialSeed }},
  fuzzRuns: {{ fuzzRuns }},
  mode: "{{ reporter }}",
};
const app = Elm.Reporter.init({ flags: flags });

// Pipe the Elm stdout port to stdout
app.ports.stdout.subscribe((str) => process.stdout.write(str));

// Export function to set the callback function when reports are finished
let finishCallback = () => console.error("finishCallback not defined yet");
app.ports.signalFinished.subscribe((code) => finishCallback(code));
exports.setCallback = (callback) => { finishCallback = callback; };

// Export function to restart the Elm reporter
exports.restart = app.ports.restart.send;

// Export function to send results to Elm
exports.sendResult = app.ports.incomingResult.send;
```

-}
worker : Ports Msg -> Program Flags Model Msg
worker ({ restart, incomingResult } as ports) =
    Platform.worker
        { init = init ports
        , update = update
        , subscriptions = \_ -> Sub.batch [ restart Restart, incomingResult IncomingResult ]
        }



-- Types


{-| Arguments passed to the reporter at startup,
such as the initial random seed, the number of fuzz runs,
and the type of reporter requested: (default)Console|Json|Junit
-}
type alias Flags =
    { initialSeed : Int
    , fuzzRuns : Int
    , mode : String
    }


{-| Main model. Exposed for usage in type definitions.
-}
type alias Model =
    { ports : Ports Msg
    , reporter : Interface
    , nbTests : Int
    , testResults : Array TestResult
    }


{-| Internal messages.
-}
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

                ( count, total ) =
                    ( Array.length allTestResults, model.nbTests )

                updatedModel =
                    { model | testResults = allTestResults }
            in
            if Array.length updatedModel.testResults == model.nbTests then
                ( updatedModel
                , Result.map (model.reporter.onResult { count = count, total = total }) testResultResult
                    |> Result.map (reportAndThenSummarize model.ports.stdout)
                    |> Result.withDefault Cmd.none
                )

            else
                ( updatedModel
                , Result.map (model.reporter.onResult { count = count, total = total }) testResultResult
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

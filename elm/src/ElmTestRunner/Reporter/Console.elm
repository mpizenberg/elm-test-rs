module ElmTestRunner.Reporter.Console exposing (implementation)

{-| Console implementation of a reporter

@docs implementation

-}

import Array exposing (Array)
import ElmTestRunner.Failure exposing (Failure)
import ElmTestRunner.Reporter.Interface exposing (Interface)
import ElmTestRunner.Result as TestResult exposing (Summary, TestResult(..))
import String.Format


{-| Provide a console implementation of a reporter, mostly for human consumption.
Require the initial random seed and number of fuzz runs.
-}
implementation : { seed : Int, fuzzRuns : Int } -> Interface
implementation options =
    { onBegin = onBegin options
    , onResult = \_ -> Nothing
    , onEnd = onEnd
    }


onBegin : { seed : Int, fuzzRuns : Int } -> Int -> Maybe String
onBegin { seed, fuzzRuns } nbTests =
    """
Running {{ nbTests }} tests. To reproduce these results later, run:
elm-test-rs --seed {{ seed }} --fuzz {{ fuzzRuns }} {{ files }}
"""
        |> String.Format.namedValue "nbTests" (String.fromInt nbTests)
        |> String.Format.namedValue "seed" (String.fromInt seed)
        |> String.Format.namedValue "fuzzRuns" (String.fromInt fuzzRuns)
        |> String.Format.namedValue "files" "(TODO: pass files to reporter)"
        |> Just


formatFailed : { labels : List String, todos : List String, failures : List Failure } -> String
formatFailed { labels, todos, failures } =
    """
{{ labels }}

    with todos: {{ todos }}
    with failures: {{ failures }}
"""
        |> String.Format.namedValue "labels" (formatLabels labels)
        |> String.Format.namedValue "todos" (Debug.toString todos)
        |> String.Format.namedValue "failures" (Debug.toString failures)


formatLabels : List String -> String
formatLabels =
    formatLabelsHelp []


formatLabelsHelp : List String -> List String -> String
formatLabelsHelp formattedLines labels =
    case ( formattedLines, labels ) of
        ( _, [] ) ->
            String.join "\n" formattedLines

        -- First is the test name
        ( [], testName :: location ) ->
            formatLabelsHelp [ "X " ++ testName ] location

        ( _, loc :: location ) ->
            formatLabelsHelp (("| " ++ loc) :: formattedLines) location


onEnd : Array String -> Array TestResult -> Maybe String
onEnd logs testResults =
    let
        resultsDetails =
            Array.indexedMap (details logs) testResults
                |> Array.toList
                |> List.filterMap identity
                |> String.join "\n"
    in
    (resultsDetails ++ "\n" ++ formatSummary (TestResult.summary testResults))
        |> Just


details : Array String -> Int -> TestResult -> Maybe String
details logs id testResult =
    case testResult of
        Passed _ ->
            Nothing

        Failed { labels, todos, failures } ->
            let
                formattedFailure =
                    formatFailed
                        { labels = labels
                        , todos = todos
                        , failures = failures
                        }
            in
            case Array.get id logs of
                Nothing ->
                    Just formattedFailure

                Just "" ->
                    Just formattedFailure

                Just errorLogs ->
                    Just (formattedFailure ++ "    with logs:\n\n" ++ errorLogs ++ "\n")


formatSummary : Summary -> String
formatSummary { nbPassed, nbFailed } =
    """
---------------------------------

TEST RUN {{ result }}

Duration: {{ duration }} ms
Passed:   {{ passed }}
Failed:   {{ failed }}

"""
        |> String.Format.namedValue "result" (summaryTitle (nbFailed > 0))
        |> String.Format.namedValue "duration" "(TODO: measure durations)"
        |> String.Format.namedValue "passed" (String.fromInt nbPassed)
        |> String.Format.namedValue "failed" (String.fromInt nbFailed)


summaryTitle : Bool -> String
summaryTitle failed =
    if failed then
        "FAILED"

    else
        "PASSED"

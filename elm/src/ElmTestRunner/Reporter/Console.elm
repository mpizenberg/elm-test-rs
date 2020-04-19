module ElmTestRunner.Reporter.Console exposing (implementation)

{-| Console implementation of a reporter

@docs implementation

-}

import Array exposing (Array)
import ElmTestRunner.Reporter.Interface exposing (Interface)
import ElmTestRunner.Reporter.Progress as Progress
import ElmTestRunner.Result as TestResult exposing (Summary, TestResult(..))
import String.Format


{-| Provide a console implementation of a reporter, mostly for human consumption.
Require the initial random seed and number of fuzz runs.
-}
implementation : { seed : Int, fuzzRuns : Int } -> Interface
implementation options =
    { onBegin = onBegin options
    , onResult = onResult
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


onResult : { count : Int, total : Int } -> TestResult -> Maybe String
onResult { count, total } _ =
    Just ("\u{000D}" ++ Progress.bar { progress = count, total = total, size = 40 })


displayResult : TestResult -> Maybe String
displayResult result =
    case result of
        Passed _ ->
            Nothing

        Failed { labels, todos, failures } ->
            """
{{ labels }}

    with todos: {{ todos }}
    with failures: {{ failures }}
"""
                |> String.Format.namedValue "labels" (formatLabels labels)
                |> String.Format.namedValue "todos" (Debug.toString todos)
                |> String.Format.namedValue "failures" (Debug.toString failures)
                |> Just


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


onEnd : Array TestResult -> Maybe String
onEnd testResults =
    let
        failedResultsString =
            Array.toList testResults
                |> List.filterMap displayResult
                |> String.join ""

        summary =
            formatSummary (TestResult.summary testResults)
    in
    Just ("\n" ++ failedResultsString ++ summary)


formatSummary : Summary -> String
formatSummary { nbPassed, nbFailed } =
    """
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

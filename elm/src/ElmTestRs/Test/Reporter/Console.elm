module ElmTestRs.Test.Reporter.Console exposing (implementation)

import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Result exposing (TestResult(..))
import String.Format


implementation : { seed : Int, fuzzRuns : Int } -> Interface
implementation options =
    { onBegin = onBegin options
    , onResult = onResult
    , onEnd = always (Just "End CONSOLE report\n")
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


onResult : TestResult -> Maybe String
onResult result =
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

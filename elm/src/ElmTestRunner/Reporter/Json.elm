module ElmTestRunner.Reporter.Json exposing (implementation)

import Array exposing (Array)
import ElmTestRunner.Failure as Failure
import ElmTestRunner.Reporter.Interface exposing (Interface)
import ElmTestRunner.Result as TestResult exposing (TestResult(..))
import Json.Encode as Encode
import String.Format


implementation : { seed : Int, fuzzRuns : Int } -> Interface
implementation options =
    { onBegin = onBegin options
    , onResult = onResult
    , onEnd = onEnd
    }


onBegin : { seed : Int, fuzzRuns : Int } -> Int -> Maybe String
onBegin { seed, fuzzRuns } nbTests =
    """{"event":"runStart","testCount":"{{ nbTests }}","initialSeed":"{{ seed }}","fuzzRuns":"{{ fuzzRuns }}","paths":{{ paths }}}
"""
        |> String.Format.namedValue "nbTests" (String.fromInt nbTests)
        |> String.Format.namedValue "seed" (String.fromInt seed)
        |> String.Format.namedValue "fuzzRuns" (String.fromInt fuzzRuns)
        |> Just


onResult : TestResult -> Maybe String
onResult result =
    let
        { status, testLabels, testFailures, testDuration } =
            case result of
                Passed { labels, duration } ->
                    { status = "pass"
                    , testLabels = Encode.encode 0 (Encode.list Encode.string (List.reverse labels))
                    , testFailures = "[]"
                    , testDuration = duration
                    }

                Failed { labels, duration, todos, failures } ->
                    { status = "fail"
                    , testLabels = Encode.encode 0 (Encode.list Encode.string (List.reverse labels))
                    , testFailures = Encode.encode 0 (Encode.list Failure.encode failures)
                    , testDuration = duration
                    }
    in
    """{"event":"testCompleted","status":"{{ status }}","labels":{{ labels }},"failures":{{ failures }},"duration":"{{ duration }}"}
"""
        |> String.Format.namedValue "status" status
        |> String.Format.namedValue "labels" testLabels
        |> String.Format.namedValue "failures" testFailures
        |> String.Format.namedValue "duration" (String.fromFloat testDuration)
        |> Just


onEnd : Array TestResult -> Maybe String
onEnd results =
    let
        { totalDuration, nbPassed, nbFailed } =
            TestResult.summary results
    in
    """{"event":"runComplete","passed":"{{ passed }}","failed":"{{ failed }}","duration":"{{ duration }}","autoFail":null}
"""
        |> String.Format.namedValue "passed" (String.fromInt nbPassed)
        |> String.Format.namedValue "failed" (String.fromInt nbFailed)
        |> String.Format.namedValue "duration" (String.fromFloat totalDuration)
        |> Just

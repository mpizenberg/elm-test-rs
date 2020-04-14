module ElmTestRs.Test.Reporter.Console exposing (implementation)

import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Result exposing (TestResult(..))


implementation : Interface
implementation =
    { onBegin = always (Just "Begin CONSOLE report\n")
    , onResult = onResult
    , onEnd = always (Just "End CONSOLE report\n")
    }


onResult : TestResult -> Maybe String
onResult result =
    case result of
        Passed { labels } ->
            Just ("PASSED:" ++ String.join " / " labels ++ "\n")

        Failed { labels, todos, failures } ->
            Just ("FAILED:" ++ String.join " / " labels ++ "\n")

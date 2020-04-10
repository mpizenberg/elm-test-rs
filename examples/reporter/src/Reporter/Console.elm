module Reporter.Console exposing (implementation)

import Data
import Reporter.Interface exposing (Interface)


implementation : Interface
implementation =
    { onBegin = always (Just "Begin CONSOLE report\n")
    , onResult = onResult
    , onEnd = always (Just "End CONSOLE report\n")
    }


onResult : Data.TestResult -> Maybe String
onResult { labels, outcome } =
    case outcome of
        Data.Passed ->
            Just ("Get result PASSED:" ++ String.join " / " labels ++ "\n")

        Data.Todo ->
            Just ("Get result TODO:" ++ String.join " / " labels ++ "\n")

        Data.Failed ->
            Just ("Get result FAILED:" ++ String.join " / " labels ++ "\n")

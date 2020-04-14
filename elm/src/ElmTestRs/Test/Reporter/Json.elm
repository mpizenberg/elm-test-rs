module ElmTestRs.Test.Reporter.Json exposing (implementation)

import Array exposing (Array)
import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Json.Encode as Encode


implementation : Interface
implementation =
    { onBegin = always (Just "Begin JSON report\n")
    , onResult = always Nothing
    , onEnd = \results -> Just (summary results)
    }


summary : Array TestResult -> String
summary results =
    Encode.array TestResult.encode results
        |> Encode.encode 2

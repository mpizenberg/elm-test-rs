module Reporter.Json exposing (implementation)

import Array exposing (Array)
import Data
import Json.Encode as Encode
import Reporter.Interface exposing (Interface)


implementation : Interface
implementation =
    { onBegin = always (Just "Begin JSON report\n")
    , onResult = always Nothing
    , onEnd = \results -> Just (summary results)
    }


summary : Array Data.TestResult -> String
summary results =
    Encode.array Data.encodeTestResult results
        |> Encode.encode 0

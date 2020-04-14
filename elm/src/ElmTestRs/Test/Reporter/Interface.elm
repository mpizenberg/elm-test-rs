module ElmTestRs.Test.Reporter.Interface exposing (Interface)

import Array exposing (Array)
import ElmTestRs.Test.Result exposing (TestResult)


type alias Interface =
    { onBegin : () -> Maybe String
    , onResult : TestResult -> Maybe String
    , onEnd : Array TestResult -> Maybe String
    }

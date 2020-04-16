module ElmTestRunner.Reporter.Interface exposing (Interface)

import Array exposing (Array)
import ElmTestRunner.Result exposing (TestResult)


type alias Interface =
    { onBegin : Int -> Maybe String
    , onResult : TestResult -> Maybe String
    , onEnd : Array TestResult -> Maybe String
    }

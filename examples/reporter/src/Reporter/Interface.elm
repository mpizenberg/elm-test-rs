module Reporter.Interface exposing (Interface)

import Array exposing (Array)
import Data


type alias Interface =
    { onBegin : () -> Maybe String
    , onResult : Data.TestResult -> Maybe String
    , onEnd : Array Data.TestResult -> Maybe String
    }

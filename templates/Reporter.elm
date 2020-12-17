port module Reporter exposing (main)

import ElmTestRunner.Reporter exposing (Flags, Model, Msg)
import Json.Decode exposing (Value)


port restart : (Int -> msg) -> Sub msg


port incomingResult : (Value -> msg) -> Sub msg


port signalFinished : { exitCode : Int, testsCount : Int } -> Cmd msg


port stdout : String -> Cmd msg


main : Program Flags Model Msg
main =
    ElmTestRunner.Reporter.worker
        { restart = restart
        , incomingResult = incomingResult
        , stdout = stdout
        , signalFinished = signalFinished
        }

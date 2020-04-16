port module Reporter exposing (main)

import Internal.Reporter exposing (Flags, Model, Msg)
import Json.Decode exposing (Value)

port restart : (Int -> msg) -> Sub msg
port incomingResult : (Value -> msg) -> Sub msg
port signalFinished : Int -> Cmd msg
port stdout : String -> Cmd msg

main : Program Flags Model Msg
main =
    Internal.Reporter.worker
        { restart = restart
        , incomingResult = incomingResult
        , stdout = stdout
        , signalFinished = signalFinished
        }

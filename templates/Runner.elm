port module Runner exposing (main)

{{ user_imports }}
import Test
import ElmTestRs.Test.Runner exposing (Ports, Msg)
import Json.Encode as Encode exposing (Value)

port askNbTests : (Value -> msg) -> Sub msg
port sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg
port receiveRunTest : (Int -> msg) -> Sub msg
port sendResult : { type_ : String, id: Int, result : Value } -> Cmd msg

ports : Ports msg
ports =
    { askNbTests = askNbTests
    , sendNbTests = \nb -> sendNbTests { type_ = "nbTests", nbTests = nb }
    , receiveRunTest = receiveRunTest
    , sendResult = \id res -> sendResult { type_ = "result", id = id, result = res }
    }

main : ElmTestRs.Test.Runner.Program Msg
main =
    [ {{ tests }} ]
        |> Test.concat
        |> ElmTestRs.Test.Runner.worker ports

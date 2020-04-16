port module Runner exposing (main)

{{ user_imports }}
import Test
import ElmTestRs.Test.Runner exposing (Ports, Msg)
import Json.Encode exposing (Value)

port askNbTests : (Value -> msg) -> Sub msg
port sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg
port receiveRunTest : (Int -> msg) -> Sub msg
port sendResult : Value -> Cmd msg

ports : Ports msg
ports =
    { askNbTests = askNbTests
    , sendNbTests = sendNbTests
    , receiveRunTest = receiveRunTest
    , sendResult = sendResult
    }

main : ElmTestRs.Test.Runner.Program Msg
main =
    [ {{ tests }} ]
        |> Test.concat
        |> ElmTestRs.Test.Runner.worker ports

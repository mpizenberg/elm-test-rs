port module Runner exposing (main)

{{ user_imports }}

import ElmTestRunner.Runner exposing (Flags, Model, Msg)
import Json.Encode exposing (Value)
import Test exposing (Test)


port askNbTests : (Value -> msg) -> Sub msg


port sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg


port receiveRunTest : (Int -> msg) -> Sub msg


port sendResult : { type_ : String, id : Int, result : Value } -> Cmd msg


testsForModule : List Test
testsForModule =
    [ {{ tests }}
    ]


main : Program Flags Model Msg
main =
    testsForModule
        |> Test.concat
        |> ElmTestRunner.Runner.worker
            { askNbTests = askNbTests
            , sendNbTests = \nb -> sendNbTests { type_ = "nbTests", nbTests = nb }
            , receiveRunTest = receiveRunTest
            , sendResult = \id res -> sendResult { type_ = "result", id = id, result = res }
            }

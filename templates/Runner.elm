port module Runner exposing (main)

{{ imports }}

import ElmTestRunner.Runner exposing (Flags, Model, Msg)
import Json.Encode exposing (Value)
import Test exposing (Test)
import Test.RunnerV2


port askTestsCount : (Value -> msg) -> Sub msg


port sendTestsCount : { kind : String, testsCount : Int } -> Cmd msg


port receiveRunTest : (Int -> msg) -> Sub msg


port sendResult : { id : Int, result : Value } -> Cmd msg


{-| Returns `Just value` if `value` is a `Test`, otherwise `Nothing`.
-}
check : a -> Maybe Test
check =
    Test.RunnerV2.identifyTest


tests : List Test
tests =
    [ {{ potential_tests }} ]
        |> List.filterMap identity


main : Program Flags Model Msg
main =
    let
        concatenatedTest =
            case tests of
                [] ->
                    Nothing

                _ ->
                    Just (Test.concat tests)
    in
    concatenatedTest
        |> ElmTestRunner.Runner.worker
            { askTestsCount = askTestsCount
            , sendTestsCount = sendTestsCount
            , receiveRunTest = receiveRunTest
            , sendResult = sendResult
            }

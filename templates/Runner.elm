port module Runner exposing (main)

{{ imports }}

import ElmTestRunner.Runner exposing (Flags, Model, Msg)
import Json.Encode exposing (Value)
import Test exposing (Test)


port askTestsCount : (Value -> msg) -> Sub msg


port sendTestsCount : Int -> Cmd msg


port receiveRunTest : (Int -> msg) -> Sub msg


port sendResult : { id : Int, result : Value } -> Cmd msg


{-| The implementation of this function will be replaced in the generated JS
with a version that returns `Just value` if `value` is a `Test`, otherwise `Nothing`.
If you rename or change this function you also need to update the regex that looks for it.
-}
check : a -> Maybe Test
check =
    checkHelperReplaceMe___


checkHelperReplaceMe___ : a -> b
checkHelperReplaceMe___ _ =
    Debug.todo """The regex for replacing this Debug.todo with some real code must have failed since you see this message!

Please report this bug: https://github.com/mpizenberg/elm-test-rs/issues/new
"""


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

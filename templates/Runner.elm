port module Runner exposing (main)

{{ user_imports }}

import ElmTestRunner.Runner exposing (Flags, Model, Msg)
import Json.Encode exposing (Value)
import Test exposing (Test)


port askNbTests : (Value -> msg) -> Sub msg


port sendNbTests : { type_ : String, nbTests : Int } -> Cmd msg


port receiveRunTest : (Int -> msg) -> Sub msg


port sendResult : { type_ : String, id : Int, result : Value } -> Cmd msg


{-| The implementation of this function will be replaced in the generated JS
with a version that returns `Just value` if `value` is a `Test`, otherwise `Nothing`.
If you rename or change this function you also need to update the regex that looks for it.
-}
check : a -> Maybe Test
check =
    checkHelperReplaceMe___


checkHelperReplaceMe___ : a -> b
checkHelperReplaceMe___ _ =
    Debug.todo "The regex for replacing this Debug.todo with some real code must have failed since you see this message!\n\nPlease report this bug: https://github.com/rtfeldman/node-test-runner/issues/new\n"


testsForModule : List { module_ : String, maybeTests : List (Maybe Test) }
testsForModule =
    [ {{ tests }}
    ]


main : Program Flags Model Msg
main =
    testsForModule
        |> List.filterMap
            (\{ module_, maybeTests } ->
                case maybeTests |> List.filterMap (\x -> x) of
                    [] ->
                        Nothing

                    tests ->
                        Just (Test.describe module_ tests)
            )
        |> Test.concat
        |> ElmTestRunner.Runner.worker
            { askNbTests = askNbTests
            , sendNbTests = \nb -> sendNbTests { type_ = "nbTests", nbTests = nb }
            , receiveRunTest = receiveRunTest
            , sendResult = \id res -> sendResult { type_ = "result", id = id, result = res }
            }

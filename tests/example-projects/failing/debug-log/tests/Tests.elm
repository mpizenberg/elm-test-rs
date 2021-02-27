module Tests exposing (..)

import Expect
import Question
import Test exposing (Test)


suite : Test
suite =
    Test.describe "Question"
        [ Test.test "answer" <|
            \_ ->
                Question.answer "What is the Answer to the Ultimate Question of Life, The Universe, and Everything?"
                    |> Expect.equal 42
        ]

module Tests exposing (..)

import Expect
import Fuzz
import Question
import Test exposing (Test)


suite : Test
suite =
    Test.fuzzWith
        { runs = 100
        , coverage =
            Test.reportCoverage
                [ ( "low", \n -> n == 1 )
                , ( "high", \n -> n == 20 )
                , ( "in between", \n -> n > 1 && n < 20 )
                , ( "outside", \n -> n < 1 || n > 20 )
                ]
        }
        (Fuzz.intRange 1 20)
        "Failing test with coverage report"
        (\n -> Expect.fail "Test fails no matter what")

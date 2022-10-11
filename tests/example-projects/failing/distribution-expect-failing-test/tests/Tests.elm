module Tests exposing (..)

import Expect
import Fuzz
import Question
import Test exposing (Test)
import Test.Distribution


suite : Test
suite =
    Test.fuzzWith
        { runs = 100
        , distribution =
            Test.expectDistribution
                [ ( Test.Distribution.atLeast 4, "low", \n -> n == 1 )
                , ( Test.Distribution.atLeast 4, "high", \n -> n == 20 )
                , ( Test.Distribution.atLeast 80, "in between", \n -> n > 1 && n < 20 )
                , ( Test.Distribution.zero, "outside", \n -> n < 1 || n > 20 )
                , ( Test.Distribution.moreThanZero, "one", \n -> n == 1 )
                ]
        }
        (Fuzz.intRange 1 20)
        "Will fail because of distribution demands not met"
        (\n -> Expect.fail "This test should fail")

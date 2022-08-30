module Tests exposing (..)

import Expect
import Fuzz
import Question
import Test exposing (Test)
import Test.Coverage


suite : Test
suite =
    Test.fuzzWith
        { runs = 100
        , coverage =
            Test.expectCoverage
                [ ( Test.Coverage.atLeast 4, "low", \n -> n == 1 )
                , ( Test.Coverage.atLeast 4, "high", \n -> n == 20 )
                , ( Test.Coverage.atLeast 80, "in between", \n -> n > 1 && n < 20 )
                , ( Test.Coverage.zero, "outside", \n -> n < 1 || n > 20 )
                , ( Test.Coverage.zero, "one", \n -> n == 1 )
                ]
        }
        (Fuzz.intRange 1 20)
        "Will fail because of coverage demands not met"
        (\n -> Expect.pass)

module Tests exposing (..)

import Expect
import Fuzz
import Test exposing (Test)
import Test.Coverage


suite : Test
suite =
    Test.fuzzWith
        { runs = 10000
        , coverage =
            Test.reportCoverage
                [ ( "low", \n -> n == 1 )
                , ( "high", \n -> n == 20 )
                , ( "in between", \n -> n > 1 && n < 20 )
                , ( "outside", \n -> n < 1 || n > 20 )
                ]
        }
        (Fuzz.intRange 1 20)
        "Int range boundaries"
        (\n -> Expect.pass)

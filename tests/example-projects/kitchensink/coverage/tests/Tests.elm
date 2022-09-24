module Tests exposing (suite)

import Expect
import Fuzz
import Question
import Test exposing (Test)
import Test.Coverage


suite : Test
suite =
    Test.concat
        [ reportCoveragePassing
        , reportCoverageFailing
        , expectCoveragePassing
        , expectCoverageFailingCoverage
        , expectCoverageFailingTest
        ]


reportCoveragePassing : Test
reportCoveragePassing =
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
        "reportCoverage: passing"
        (\n -> Expect.pass)


reportCoverageFailing : Test
reportCoverageFailing =
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
        "reportCoverage: failing"
        (\n -> Expect.fail "Test fails no matter what")


expectCoveragePassing : Test
expectCoveragePassing =
    Test.fuzzWith
        { runs = 100
        , coverage =
            Test.expectCoverage
                [ ( Test.Coverage.atLeast 4, "low", \n -> n == 1 )
                , ( Test.Coverage.atLeast 4, "high", \n -> n == 20 )
                , ( Test.Coverage.atLeast 80, "in between", \n -> n > 1 && n < 20 )
                , ( Test.Coverage.zero, "outside", \n -> n < 1 || n > 20 )
                , ( Test.Coverage.moreThanZero, "one", \n -> n == 1 )
                ]
        }
        (Fuzz.intRange 1 20)
        "expectCoverage: passing"
        (\n -> Expect.pass)


expectCoverageFailingCoverage : Test
expectCoverageFailingCoverage =
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
        "expectCoverage: failing because of coverage"
        (\n -> Expect.pass)


expectCoverageFailingTest : Test
expectCoverageFailingTest =
    Test.fuzzWith
        { runs = 100
        , coverage =
            Test.expectCoverage
                [ ( Test.Coverage.atLeast 4, "low", \n -> n == 1 )
                , ( Test.Coverage.atLeast 4, "high", \n -> n == 20 )
                , ( Test.Coverage.atLeast 80, "in between", \n -> n > 1 && n < 20 )
                , ( Test.Coverage.zero, "outside", \n -> n < 1 || n > 20 )
                , ( Test.Coverage.moreThanZero, "one", \n -> n == 1 )
                ]
        }
        (Fuzz.intRange 1 20)
        "expectCoverage: failing because of test"
        (\n -> Expect.fail "This test should fail")

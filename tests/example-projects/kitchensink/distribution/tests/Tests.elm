module Tests exposing (suite)

import Expect
import Fuzz
import Question
import Test exposing (Test)
import Test.Distribution


suite : Test
suite =
    Test.concat
        [ reportDistributionPassing
        , reportDistributionFailing
        , expectDistributionPassing
        , expectDistributionFailingDistribution
        , expectDistributionFailingTest
        ]


reportDistributionPassing : Test
reportDistributionPassing =
    Test.fuzzWith
        { runs = 10000
        , distribution =
            Test.reportDistribution
                [ ( "low", \n -> n == 1 )
                , ( "high", \n -> n == 20 )
                , ( "in between", \n -> n > 1 && n < 20 )
                , ( "outside", \n -> n < 1 || n > 20 )
                ]
        }
        (Fuzz.intRange 1 20)
        "reportDistribution: passing"
        (\n -> Expect.pass)


reportDistributionFailing : Test
reportDistributionFailing =
    Test.fuzzWith
        { runs = 100
        , distribution =
            Test.reportDistribution
                [ ( "low", \n -> n == 1 )
                , ( "high", \n -> n == 20 )
                , ( "in between", \n -> n > 1 && n < 20 )
                , ( "outside", \n -> n < 1 || n > 20 )
                ]
        }
        (Fuzz.intRange 1 20)
        "reportDistribution: failing"
        (\n -> Expect.fail "Test fails no matter what")


expectDistributionPassing : Test
expectDistributionPassing =
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
        "expectDistribution: passing"
        (\n -> Expect.pass)


expectDistributionFailingDistribution : Test
expectDistributionFailingDistribution =
    Test.fuzzWith
        { runs = 100
        , distribution =
            Test.expectDistribution
                [ ( Test.Distribution.atLeast 4, "low", \n -> n == 1 )
                , ( Test.Distribution.atLeast 4, "high", \n -> n == 20 )
                , ( Test.Distribution.atLeast 80, "in between", \n -> n > 1 && n < 20 )
                , ( Test.Distribution.zero, "outside", \n -> n < 1 || n > 20 )
                , ( Test.Distribution.zero, "one", \n -> n == 1 )
                ]
        }
        (Fuzz.intRange 1 20)
        "expectDistribution: failing because of distribution"
        (\n -> Expect.pass)


expectDistributionFailingTest : Test
expectDistributionFailingTest =
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
        "expectDistribution: failing because of test"
        (\n -> Expect.fail "This test should fail")

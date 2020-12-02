module ElmTestRunner.Reporter.Progress exposing (bar)

{-| Show progress of tests

@docs bar

-}


{-| Display a progress bar
-}
bar : { progress : Int, total : Int, size : Int } -> String
bar { progress, total, size } =
    let
        progressSize =
            size * progress // total
    in
    String.concat
        [ "["
        , String.repeat progressSize "∎"
        , String.repeat (size - progressSize) " "
        , "]"
        ]

module Question exposing (answer)


answer : String -> Int
answer question =
    let
        _ =
            Debug.log "The question was" question
    in
    if question == "What is the Answer to the Ultimate Question of Life, The Universe, and Everything?" then
        43

    else
        0

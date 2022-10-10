@test "run update" {
    target/debug/mist update
}

@test "search with no results" {
    run ! target/debug/mist search 'nonexistent'
}

@test "list with no results" {
    run ! target/debug/mist list 'nonexistent'
}
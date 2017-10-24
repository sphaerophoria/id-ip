error_chain!{
    errors {
        ParseMacAddrErr(s: String) {
            display("Failed to parse mac addr: {}", s)
        }
    }
}

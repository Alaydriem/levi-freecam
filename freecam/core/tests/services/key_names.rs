use freecam_core::services::KeyNames;

#[test]
fn common_keys_have_names() {
    let cases = [(0x11, "Ctrl"), (0x45, "E"), (0x2E, "Del"), (0x0D, "Enter"), (0x50, "P"), (0x42, "B"), (0x70, "F1")];
    for (code, expected) in cases {
        assert_eq!(KeyNames::name(code), expected);
    }
}

#[test]
fn unnamed_key_gets_hex_label() {
    assert_eq!(KeyNames::name(0xBA), "Key 0xBA");
}

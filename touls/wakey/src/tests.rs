use super::*;

#[test]
fn test_valid_mac_to_array() {
    let letter_mac = "aa:bb:cc:dd:ee:ff";
    assert_eq!(
        mac_to_array(letter_mac),
        Ok([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
    );

    let number_mac = "11:22:33:44:55:66";
    assert_eq!(
        mac_to_array(number_mac),
        Ok([0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
    );

    let mixed_mac = "a7:8b:99:cc:d0:1e";
    assert_eq!(
        mac_to_array(mixed_mac),
        Ok([0xa7, 0x8b, 0x99, 0xcc, 0xd0, 0x1e])
    );

    let broadcast_mac = "ff:ff:ff:ff:ff:ff";
    assert_eq!(
        mac_to_array(broadcast_mac),
        Ok([0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
    );
}

#[test]
fn test_invalid_mac_to_array() { 
    let empty_string = "";
    assert!(mac_to_array(empty_string).is_err());

    let random_string = "test";
    assert!(mac_to_array(random_string).is_err());

    let wrong_format_1 = "aaa:bb:cc:dd:ee:ff";
    assert!(mac_to_array(wrong_format_1).is_err());

    let wrong_format_2 = "11::22:33:44:55:66";
    assert!(mac_to_array(wrong_format_2).is_err());

    let wrong_format_3 = "11:22:33";
    assert!(mac_to_array(wrong_format_3).is_err());

    let wrong_format_4 = "11:22:33:44:55:66:77";
    assert!(mac_to_array(wrong_format_4).is_err());

    let wrong_format_5 = "ll:aa:bb:zz:00:aa";
    assert!(mac_to_array(wrong_format_5).is_err());
}

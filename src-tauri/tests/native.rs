use yiyin_desktop::{dto::ExternalDestinationDto, native::external_url};

#[test]
fn external_destinations_are_closed_and_allowlisted_in_rust() {
    assert_eq!(
        external_url(ExternalDestinationDto::Repository),
        "https://github.com/ggchivalrous/yiyin"
    );
    assert_eq!(
        external_url(ExternalDestinationDto::Issues),
        "https://github.com/ggchivalrous/yiyin/issues"
    );
    assert_eq!(
        external_url(ExternalDestinationDto::CurrentRelease),
        "https://github.com/ggchivalrous/yiyin/releases/tag/v1.6.0"
    );
    assert_eq!(
        external_url(ExternalDestinationDto::BilibiliProfile),
        "https://space.bilibili.com/94829489"
    );
    assert_eq!(
        external_url(ExternalDestinationDto::BilibiliFeedback),
        "https://message.bilibili.com/#/whisper/mid94829489"
    );
}

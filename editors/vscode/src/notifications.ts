export type RateLimitNotificationKind = "GitHub";

export type RateLimitNotification = {
	kind: RateLimitNotificationKind;
	value: any;
};

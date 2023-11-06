# Relay Banhammer

Relay Banhammer is a validation server for `nostr-rs-relay`. 
For any message sent to your `nostr-rs-relay`, the relay will poll the validation server which will match against a ban list to accept or reject the event. 

The banhammer currently supports ban on : 
- sender ip
- pubkey
- content
- tags



## Configure

The ban list must be declared in a `yaml` file. 

| Field | Type | Description |
|-------|------|-------------|
| content | string | The reference value the validator will use, an npub if user ban. |
| regex | boolean | Interpret content as a regex. |
| date  | string | The creation date of the ban.  |
| ban_type | One of `IP`,`CONTENT`,`TAG`,`USER` | The type of ban to be applied. |

e.g:

```yaml
  - ban_type: CONTENT
  content: hello
  regex: false
  date: "2023-01-01T00:01:23"
  - ban_type: TAG
  content: world
  regex: false
  date: "2023-01-01T00:01:23"
  # use npub for user ban
  - ban_type: USER
  content: npub1gn5ha3qaxqgtvxhfdwsyt38s2sdu8jxmad92c0zuhfrthmnq9s5sxhfe6u
  regex: false
  date: "2023-01-01T00:01:23"
  - ban_type: IP
  content: 192.168.0.255
  regex: false
  date: "2023-01-01T00:01:23"
```



## CLI 

The service comes with an additional CLI program to help in basic management of your banlist.
# GetAccountConfig200ResponsePlansInner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**plan_id** | **String** | The plan id as defined in Global Catalog | 
**usage_allocation_seconds** | **i32** | The maximum sum of allowable usage allocation across all instances in the account. | 
**backends** | **Vec<String>** | List of backends to allow the account to assign for instances. | 
**max_ttl** | Option<**i32**> | The maximum time (in seconds) for session to run, subject to plan limits. | [optional]
**active_ttl** | Option<**i32**> | The remaining time (in seconds) for the session to be in the active state while jobs are running. Must be less than or equal to max ttl. | [optional]
**interactive_ttl** | Option<**i32**> | The maximum time (in seconds) between jobs to keep the session active. Must be less than or equal to active ttl. | [optional]
**unallocated_usage_seconds** | Option<**i32**> | The remaining usage allocation that can still be allocated to instances. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



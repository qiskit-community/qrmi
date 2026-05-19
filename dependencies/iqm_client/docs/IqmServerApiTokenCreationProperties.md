# IqmServerApiTokenCreationProperties

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**activated** | Option<**bool**> | If `true` or omitted, the created token is activated immediately and the refresh token is invalidated. If `false`, the created token needs to be activated using the `/tokens/current/activate` endpoint. | [optional][default to true]
**expires_in_seconds** | **i32** |  | 
**refreshing_expires_in_seconds** | **i32** | Refresh token expiration time in seconds. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



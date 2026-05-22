# IqmServerApiToken

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**activated_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**expires_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**id** | **uuid::Uuid** | ID of the API token | 
**last_characters** | **String** | The suffix of the token to help in identifying it. | 
**refresh_token** | Option<[**models::IqmServerApiTokenRefreshInfo**](IqmServerApiTokenRefreshInfo.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



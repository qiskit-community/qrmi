# \TagsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_tags**](TagsApi.md#list_tags) | **GET** /v1/tags | List tags



## list_tags

> models::ListTags200Response list_tags(r#type, search, ibm_api_version)
List tags

Search and list the tags of jobs.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | **String** | Searches for tags in the specified type. | [required] |[default to job]
**search** | **String** | Used for searching tags. | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::ListTags200Response**](list_tags_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


# \SessionsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_session**](SessionsApi.md#create_session) | **POST** /v1/sessions | Create a session
[**delete_session_close**](SessionsApi.md#delete_session_close) | **DELETE** /v1/sessions/{id}/close | Close job session
[**get_session_information**](SessionsApi.md#get_session_information) | **GET** /v1/sessions/{id} | Get a session
[**update_session_state**](SessionsApi.md#update_session_state) | **PATCH** /v1/sessions/{id} | Update a session



## create_session

> models::CreateSession200Response create_session(ibm_api_version, create_session_request)
Create a session

Create a session

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**create_session_request** | Option<[**CreateSessionRequest**](CreateSessionRequest.md)> |  |  |

### Return type

[**models::CreateSession200Response**](create_session_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_session_close

> delete_session_close(id, ibm_api_version)
Close job session

Closes the runtime session

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Session Id | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_session_information

> models::CreateSession200Response get_session_information(id, ibm_api_version)
Get a session

Get a session

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Job Session ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::CreateSession200Response**](create_session_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_session_state

> update_session_state(id, ibm_api_version, update_session_state_request)
Update a session

Update a session

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Job Session ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**update_session_state_request** | Option<[**UpdateSessionStateRequest**](UpdateSessionStateRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


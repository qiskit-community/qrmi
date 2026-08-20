# \BackendsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_backend_configuration**](BackendsApi.md#get_backend_configuration) | **GET** /v1/backends/{id}/configuration | Get backend configuration
[**get_backend_defaults**](BackendsApi.md#get_backend_defaults) | **GET** /v1/backends/{id}/defaults | Get backend default settings
[**get_backend_properties**](BackendsApi.md#get_backend_properties) | **GET** /v1/backends/{id}/properties | Get backend properties
[**get_backend_status**](BackendsApi.md#get_backend_status) | **GET** /v1/backends/{id}/status | Get backend status
[**list_backends**](BackendsApi.md#list_backends) | **GET** /v1/backends | List your backends



## get_backend_configuration

> std::collections::HashMap<String, serde_json::Value> get_backend_configuration(id, ibm_api_version)
Get backend configuration

Returns the configuration for the specified backend.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an available backend | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_backend_defaults

> std::collections::HashMap<String, serde_json::Value> get_backend_defaults(id, ibm_api_version)
Get backend default settings

Returns the defaults for the specified backend. Simulator backends may not support this.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an available backend | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_backend_properties

> std::collections::HashMap<String, serde_json::Value> get_backend_properties(id, ibm_api_version, updated_before)
Get backend properties

Returns the properties for the specified backend. Simulator backends may not support this.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an available backend | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**updated_before** | Option<**String**> | Returns properties with last_update_date before the given time |  |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_backend_status

> models::BackendStatusResponse get_backend_status(id, ibm_api_version)
Get backend status

Returns the status for the specified backend ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an available backend | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::BackendStatusResponse**](BackendStatusResponse.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_backends

> models::ListBackends200Response list_backends(ibm_api_version)
List your backends

Returns a list of all the backends your service instance has access to. <br/><br/> This endpoint returns different response schemas depending on the value of the `IBM-API-Version` header.<br/><br/> - If `IBM-API-Version` is `2024-01-01` or omitted, the response will follow `BackendsResponse`.<br/> - If `IBM-API-Version` is `2025-01-01` or greater, the response will follow `BackendsResponseV2`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |

### Return type

[**models::ListBackends200Response**](list_backends_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


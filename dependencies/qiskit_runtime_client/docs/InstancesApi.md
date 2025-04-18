# \InstancesApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_instance**](InstancesApi.md#get_instance) | **GET** /v1/instance | Get current instance details
[**get_instance_configuration**](InstancesApi.md#get_instance_configuration) | **GET** /v1/instances/configuration | Get instance configuration
[**replace_instance_data**](InstancesApi.md#replace_instance_data) | **PUT** /v1/instances/configuration | Update instance configuration



## get_instance

> models::GetInstance200Response get_instance(ibm_api_version)
Get current instance details

Returns the details of the current logged in instance, using CRN from the request header.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::GetInstance200Response**](get_instance_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_instance_configuration

> models::GetInstanceConfiguration200Response get_instance_configuration(ibm_api_version)
Get instance configuration

Returns the configuration for the specified instance e.g. instance limit in seconds, using CRN from the request header.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::GetInstanceConfiguration200Response**](get_instance_configuration_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## replace_instance_data

> replace_instance_data(ibm_api_version, instance_update)
Update instance configuration

Update the configuration for the specified instance e.g. instance limit in seconds, using CRN from context params of the request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**instance_update** | Option<[**ReplaceInstanceDataRequest**](ReplaceInstanceDataRequest.md)> | Request body for updating a specified instance configuration. |  |

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


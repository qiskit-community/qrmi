# \AnalyticsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**analytics_filters**](AnalyticsApi.md#analytics_filters) | **GET** /v1/analytics/filters | Get analytics filters
[**analytics_usage**](AnalyticsApi.md#analytics_usage) | **GET** /v1/analytics/usage | Get usage analytics
[**get_usage_analytics_grouped**](AnalyticsApi.md#get_usage_analytics_grouped) | **GET** /v1/analytics/usage_grouped | Get usage analytics grouped
[**get_usage_analytics_grouped_by_date**](AnalyticsApi.md#get_usage_analytics_grouped_by_date) | **GET** /v1/analytics/usage_grouped_by_date | Get usage analytics grouped by date



## analytics_filters

> models::AnalyticsFilters200Response analytics_filters(ibm_api_version, instance)
Get analytics filters

Get analytics filters

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**instance** | Option<[**Vec<String>**](String.md)> |  |  |

### Return type

[**models::AnalyticsFilters200Response**](analytics_filters_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## analytics_usage

> models::AnalyticsUsage200Response analytics_usage(ibm_api_version, instance, interval_start, interval_end, backend, user_id, simulators)
Get usage analytics

Get usage analytics

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**instance** | Option<[**Vec<String>**](String.md)> |  |  |
**interval_start** | Option<**String**> |  |  |
**interval_end** | Option<**String**> |  |  |
**backend** | Option<[**Vec<String>**](String.md)> |  |  |
**user_id** | Option<[**Vec<String>**](String.md)> |  |  |
**simulators** | Option<**bool**> | Include simulators |  |[default to true]

### Return type

[**models::AnalyticsUsage200Response**](analytics_usage_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_usage_analytics_grouped

> models::GetUsageAnalyticsGrouped200Response get_usage_analytics_grouped(group_by, ibm_api_version, instance, interval_start, interval_end, backend, user_id, simulators)
Get usage analytics grouped

Get usage analytics grouped

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_by** | **String** | key to group usage by | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**instance** | Option<[**Vec<String>**](String.md)> |  |  |
**interval_start** | Option<**String**> | start date |  |
**interval_end** | Option<**String**> | end date |  |
**backend** | Option<[**Vec<String>**](String.md)> | backend to filter by |  |
**user_id** | Option<[**Vec<String>**](String.md)> |  |  |
**simulators** | Option<**bool**> | Include simulators |  |[default to true]

### Return type

[**models::GetUsageAnalyticsGrouped200Response**](get_usage_analytics_grouped_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_usage_analytics_grouped_by_date

> models::GetUsageAnalyticsGroupedByDate200Response get_usage_analytics_grouped_by_date(group_by, ibm_api_version, instance, interval_start, interval_end, backend, user_id, simulators)
Get usage analytics grouped by date

Get usage analytics grouped by date

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_by** | **String** |  | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**instance** | Option<[**Vec<String>**](String.md)> |  |  |
**interval_start** | Option<**String**> |  |  |
**interval_end** | Option<**String**> |  |  |
**backend** | Option<[**Vec<String>**](String.md)> |  |  |
**user_id** | Option<[**Vec<String>**](String.md)> |  |  |
**simulators** | Option<**bool**> | Include simulators |  |[default to true]

### Return type

[**models::GetUsageAnalyticsGroupedByDate200Response**](get_usage_analytics_grouped_by_date_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


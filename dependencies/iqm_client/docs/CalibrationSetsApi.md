# \CalibrationSetsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_calibration_set_v1**](CalibrationSetsApi.md#get_calibration_set_v1) | **GET** /api/v1/calibration-sets/{qc}/{cal_set} | Get calibration set
[**get_dynamic_quantum_architecture_v1**](CalibrationSetsApi.md#get_dynamic_quantum_architecture_v1) | **GET** /api/v1/calibration-sets/{qc}/{cal_set}/dynamic-quantum-architecture | Get dynamic quantum architecture
[**get_quality_metrics_v1**](CalibrationSetsApi.md#get_quality_metrics_v1) | **GET** /api/v1/calibration-sets/{qc}/{cal_set}/metrics | Get calibration set quality metrics



## get_calibration_set_v1

> get_calibration_set_v1(qc, cal_set)
Get calibration set

 Returns the specified quantum computer’s calibration set and its related observations by calibration set ID. Alternatively, the keyword `default` can be used to retrieve the current default calibration set. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**cal_set** | **String** | Calibration set id or `default` | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dynamic_quantum_architecture_v1

> get_dynamic_quantum_architecture_v1(qc, cal_set)
Get dynamic quantum architecture

 Returns the dynamic quantum architecture for a calibration set identified by its calibration set ID. Alternatively, the keyword `default` can be used to retrieve the dynamic quantum architecture for the current default calibration set. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**cal_set** | **String** | Calibration set id or `default` | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_quality_metrics_v1

> get_quality_metrics_v1(qc, cal_set)
Get calibration set quality metrics

 Returns the quality metrics for a calibration set identified by its calibration set ID. Alternatively, the keyword `default` can be used to retrieve the metrics for the current default calibration set.  If the specified calibration set does not have quality metrics available, the endpoint returns a `404` response. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**cal_set** | **String** | Calibration set id or `default` | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


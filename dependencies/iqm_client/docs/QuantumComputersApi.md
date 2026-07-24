# \QuantumComputersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**book_qc_timeslot_v1**](QuantumComputersApi.md#book_qc_timeslot_v1) | **POST** /api/v1/quantum-computers/{qc}/timeslots | Book timeslots
[**get_all_qcs_v1**](QuantumComputersApi.md#get_all_qcs_v1) | **GET** /api/v1/quantum-computers | List quantum computers
[**get_qc_health_v1**](QuantumComputersApi.md#get_qc_health_v1) | **GET** /api/v1/quantum-computers/{qc}/health | Get health status
[**get_qc_limits_v1**](QuantumComputersApi.md#get_qc_limits_v1) | **GET** /api/v1/quantum-computers/{qc}/limits | Get job size limits
[**get_qc_queue_v1**](QuantumComputersApi.md#get_qc_queue_v1) | **GET** /api/v1/quantum-computers/{qc}/queue-availability | Get pay-as-you-go queue availability
[**get_qc_timeslots_v1**](QuantumComputersApi.md#get_qc_timeslots_v1) | **GET** /api/v1/quantum-computers/{qc}/timeslots | Get timeslots
[**qc_get_artifacts**](QuantumComputersApi.md#qc_get_artifacts) | **GET** /api/v1/quantum-computers/{qc}/artifacts/{artifact_type} | Get configuration artifact



## book_qc_timeslot_v1

> models::IqmServerBookTimeslotsResponse book_qc_timeslot_v1(qc, iqm_server_book_timeslots_request)
Book timeslots

Books one or more timeslots on a given quantum computer. The slot references should match values returned by the timeslot listing endpoint (`GET`). If `expected_price` is provided for a slot, the request will fail when the current price differs from the expected value. All slots are booked atomically — if any slot fails, none are booked. Credits are charged immediately upon successful booking.  **Note:** Only users with the \"scheduler\" or \"admin\" role in an organization or team can book timeslots. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**iqm_server_book_timeslots_request** | [**IqmServerBookTimeslotsRequest**](IqmServerBookTimeslotsRequest.md) |  | [required] |

### Return type

[**models::IqmServerBookTimeslotsResponse**](iqm.server.BookTimeslotsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_qcs_v1

> models::IqmServerQuantumComputerList get_all_qcs_v1()
List quantum computers

 Returns a list of available quantum computers.  Refer to the **Get configuration artifact** endpoint to retrieve configuration details for each available quantum computer. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::IqmServerQuantumComputerList**](iqm.server.QuantumComputerList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_qc_health_v1

> models::IqmServerQcHealthStatus get_qc_health_v1(qc)
Get health status

 Returns the health status of a quantum computer identified by its ID. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |

### Return type

[**models::IqmServerQcHealthStatus**](iqm.server.QcHealthStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_qc_limits_v1

> models::IqmServerQcLimits get_qc_limits_v1(qc)
Get job size limits

 Returns the job size limits of a quantum computer. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |

### Return type

[**models::IqmServerQcLimits**](iqm.server.QcLimits.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_qc_queue_v1

> models::IqmServerQcQueueOverview get_qc_queue_v1(qc, timerange_start, timerange_end)
Get pay-as-you-go queue availability

 Returns the current status of the pay-as-you-go queue for this quantum computer. Indicates the number of jobs currently queued as well as the time slots during which the queued jobs are executed. The availability windows are only accurate at the time of the request. Future user bookings will affect execution of the pay-as-you-go queue.  The start and end time of returned availability windows are always truncated to the `timerange_start` and `timerange_end` provided in the request. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**timerange_start** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Start of the time range to query. Must be a UTC timestamp in RFC3339 format with 'Z' time zone indicator. Defaults to now if not provided. |  |
**timerange_end** | Option<**chrono::DateTime<chrono::FixedOffset>**> | End of the time range to check for availability of the pay-as-you-go queue. Must be a UTC timestamp in RFC3339 format with 'Z' time zone indicator. |  |

### Return type

[**models::IqmServerQcQueueOverview**](iqm.server.QcQueueOverview.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_qc_timeslots_v1

> models::IqmServerQcTimeslots get_qc_timeslots_v1(qc, timerange_start, timerange_end)
Get timeslots

 Returns timeslots for a given quantum computer. Lists slots already booked for the caller and available for booking.  Timeslot booking is not available for mock quantum computers, and the endpoint will return an empty list of available slots for such computers. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**timerange_start** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Start of the time range to query. Must be a UTC timestamp in RFC3339 format with 'Z' time zone indicator. Defaults to now if not provided. |  |
**timerange_end** | Option<**chrono::DateTime<chrono::FixedOffset>**> | End of the time range to check for availability of the pay-as-you-go queue. Must be a UTC timestamp in RFC3339 format with 'Z' time zone indicator. |  |

### Return type

[**models::IqmServerQcTimeslots**](iqm.server.QcTimeslots.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## qc_get_artifacts

> qc_get_artifacts(qc, artifact_type)
Get configuration artifact

 Returns the contents of a configuration artifact for a quantum computer identified by its ID and artifact type.  The available artifacts depend on the Station Control version of the quantum computer. Refer to the auto-generated, quantum computer-specific API documentation for details on supported artifacts and their corresponding data formats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**artifact_type** | **String** | Quantum computer artifact type | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


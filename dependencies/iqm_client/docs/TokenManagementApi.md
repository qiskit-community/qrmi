# \TokenManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_api_token_v1**](TokenManagementApi.md#activate_api_token_v1) | **POST** /api/v1/tokens/current/activate | Activate API token
[**get_api_tokens_v1**](TokenManagementApi.md#get_api_tokens_v1) | **GET** /api/v1/tokens | List API tokens
[**refresh_api_token_v1**](TokenManagementApi.md#refresh_api_token_v1) | **POST** /api/v1/tokens/current/refresh | Refresh API token
[**revoke_api_token_v1**](TokenManagementApi.md#revoke_api_token_v1) | **POST** /api/v1/tokens/{id}/revoke | Revoke API token



## activate_api_token_v1

> models::IqmServerApiTokenActivateResult activate_api_token_v1(iqm_server_api_token_activate_request)
Activate API token

 > [!note] > This endpoint is only relevant to *integration token* users.  This endpoint activates a token previously created with the \"refresh\" endpoint. It is only useful for tokens created with the property `\"activated\": false` in the refresh request. It is idempotent and has no effect on already activated tokens.  Once the token is activated, the refresh token used to create it becomes invalid.  This endpoint enforces the active token limit. The request will fail with 403 if the user already has the maximum number of active tokens.  ## Authentication  This endpoint requires that the `Authorization` header bearer token is the same **refresh token** that was used to create the API token. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**iqm_server_api_token_activate_request** | [**IqmServerApiTokenActivateRequest**](IqmServerApiTokenActivateRequest.md) |  | [required] |

### Return type

[**models::IqmServerApiTokenActivateResult**](iqm.server.ApiTokenActivateResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_api_tokens_v1

> models::IqmServerTokenList get_api_tokens_v1()
List API tokens

 Returns a list of descriptions of the active API tokens of the current user. Does NOT return the full token values, these are only available at creation time.  Does not include refresh tokens, expired or revoked tokens, or tokens which were not activated. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::IqmServerTokenList**](iqm.server.TokenList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## refresh_api_token_v1

> models::IqmServerCreatedApiToken refresh_api_token_v1(iqm_server_api_token_refresh_request)
Refresh API token

 > [!note] > This endpoint is only relevant to *integration token* users.  This endpoint creates a new API token based on a refresh token associated with an existing API token.  If the property `\"activated\"` is set to `true` or omitted entirely, the created token is activated immediately and the refresh token used to create it is invalidated right away.  ### Two-phase activation  Because networking is unreliable, an HTTP response might fail during the refresh operation, invalidating the refresh token forever. To prevent the loss of tokens, we support [two-phase commit protocol](https://en.wikipedia.org/wiki/Two-phase_commit_protocol) for the refresh operation.  If the property `\"activated\": false` is set in the request body, the created token needs to be activated using the `/tokens/current/activate` endpoint before it can be used. The used refresh token remains valid until the new token is activated.  This avoids invalidating the refresh token before the user receives it successfully.  ## Token limit  If `\"activated\": true` (or omitted), this endpoint enforces the active token limit. The request will fail with 403 if the user already has the maximum number of active tokens.  ## Authentication  This endpoint requires that the `Authorization` header bearer token is an active **refresh token**. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**iqm_server_api_token_refresh_request** | [**IqmServerApiTokenRefreshRequest**](IqmServerApiTokenRefreshRequest.md) |  | [required] |

### Return type

[**models::IqmServerCreatedApiToken**](iqm.server.CreatedApiToken.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_api_token_v1

> models::IqmServerApiTokenRevokeResult revoke_api_token_v1(id)
Revoke API token

 Revokes the API token and its associated refresh token. Once revoked, the tokens can no longer be used for authentication.  This endpoint is idempotent and has no effect on already revoked tokens.  # Authentication This endpoint may be invoked with the regular authentication methods available in this API. It does not accept refresh tokens for authentication. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** | API token id | [required] |

### Return type

[**models::IqmServerApiTokenRevokeResult**](iqm.server.ApiTokenRevokeResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


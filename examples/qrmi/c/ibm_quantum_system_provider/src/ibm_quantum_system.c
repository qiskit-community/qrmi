/*
 * This code is part of Qiskit.
 *
 * Copyright (C) 2026 IBM, UKRI-STFC (Hartree Centre)
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

#include "qrmi.h"

extern void load_dotenv();

/*
 * Usage: ibm_quantum_system_provider <config_file> <resource_name> [filter]
 *
 * config_file    - path to qrmi_config.json
 * resource_name  - name of the dynamic resource definition (is_dynamic=true)
 * filter         - optional filter string e.g. "num_qubits=127&name=test_*"
 */
int main(int argc, char *argv[]) {

  load_dotenv();

  if (argc < 3) {
    fprintf(stderr, "Usage: %s <config_file> <resource_name> [filter]\n", argv[0]);
    return EXIT_FAILURE;
  }

  const char *config_file   = argv[1];
  const char *resource_name = argv[2];
  const char *filter        = argc >= 4 ? argv[3] : NULL;

  QrmiConfig *config = qrmi_config_load(config_file);
  if (config == NULL) {
    const char *err = qrmi_get_last_error();
    fprintf(stderr, "qrmi_config_load() failed: %s\n", err);
    qrmi_string_free((char *)err);
    return EXIT_FAILURE;
  }

  QrmiResourceDef *def = qrmi_config_resource_def_get(config, resource_name);
  if (def == NULL) {
    fprintf(stderr, "Resource '%s' not found in config\n", resource_name);
    qrmi_config_free(config);
    return EXIT_FAILURE;
  }

  QrmiResourceProvider *provider = qrmi_provider_new(QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM, &def->environments);
  qrmi_config_resource_def_free(def);
  qrmi_config_free(config);

  if (provider == NULL) {
    const char *err = qrmi_get_last_error();
    fprintf(stderr, "qrmi_provider_new() failed: %s\n", err);
    qrmi_string_free((char *)err);
    return EXIT_FAILURE;
  }

  QrmiQuantumResources resources = {0};
  QrmiReturnCode rc = qrmi_provider_resources(provider, filter, &resources);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    for (size_t i = 0; i < resources.length; i++) {
      char *id = NULL;
      qrmi_resource_id(resources.resources[i], &id);
      printf("resource: %s\n", id);

      bool is_accessible = false;
      rc = qrmi_resource_is_accessible(resources.resources[i], &is_accessible);
      if (rc == QRMI_RETURN_CODE_SUCCESS) {
        if (is_accessible) {
          fprintf(stdout, "%s can be accessed.\n", id);
        } else {
          fprintf(stderr, "%s cannot be accessed.\n", id);
        }
      } else {
        const char *last_error = qrmi_get_last_error();
        fprintf(stderr, "qrmi_resource_is_accessible() failed. %s\n", last_error);
        qrmi_string_free((char *)last_error);
      }
      qrmi_string_free(id);
    }
    qrmi_provider_resources_free(&resources);
  }

  qrmi_provider_free(provider);
  return EXIT_SUCCESS;
}

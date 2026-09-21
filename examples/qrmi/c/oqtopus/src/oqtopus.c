/*
 * (C) Copyright IBM 2026.
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
extern const char *read_file(const char *);

int main(int argc, char *argv[]) {

  if (argc != 2) {
    fprintf(stderr, "oqtopus <device_id>\n");
    return EXIT_SUCCESS;
  }

  load_dotenv();

  QrmiQuantumResource *qrmi =
      qrmi_resource_new(argv[1], QRMI_RESOURCE_TYPE_OQTOPUS);
  if (!qrmi) {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "Failed to create QRMI for %s. %s (%d)\n", argv[1],
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
    return EXIT_FAILURE;
  }

  QrmiReturnCode rc = QRMI_RETURN_CODE_SUCCESS;
  char *resource_id = NULL;
  rc = qrmi_resource_id(qrmi, &resource_id);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    QrmiResourceType resource_type;
    rc = qrmi_resource_type(qrmi, &resource_type);
    if (rc == QRMI_RETURN_CODE_SUCCESS) {
      const char *resource_type_str =
          qrmi_config_resource_type_to_str(resource_type);
      fprintf(stdout, "Selected resource: id=%s type=%s\n", resource_id,
              resource_type_str);
    }
    qrmi_string_free(resource_id);
  }

  bool is_accessible = false;
  rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    if (is_accessible == false) {
      fprintf(stderr, "%s cannot be accessed.\n", argv[1]);
      goto error;
    }
  } else {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_is_accessible() failed. %s (%d)\n",
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
    goto error;
  }
  fprintf(stdout, "accessible: %d\n", is_accessible);

  qrmi_resource_free(qrmi);

  return EXIT_SUCCESS;

error:
  qrmi_resource_free(qrmi);
  return EXIT_FAILURE;
}

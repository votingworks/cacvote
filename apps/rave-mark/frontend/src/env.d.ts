declare namespace NodeJS {
  export interface ProcessEnv {
    IS_INTEGRATION_TEST?: string;
    REACT_APP_BALLOT_PRINTER_NAME?: string;
    REACT_APP_MAILING_LABEL_PRINTER_NAME?: string;
  }
}

declare namespace NodeJS {
  export interface ProcessEnv {
    readonly CI?: string;
    readonly NODE_ENV: 'development' | 'production' | 'test';
    readonly IS_INTEGRATION_TEST?: string;
    readonly USABILITY_TEST_ELECTION_PATH?: string;
    readonly USABILITY_TEST_SKIP_REGISTRATION?: string;
    readonly USABILITY_TEST_EXPIRATION_MINUTES?: string;
    readonly USABILITY_TEST_EXPIRATION_TYPE?: string;
    readonly BASE_PORT?: string;
    readonly PORT?: string;
    readonly VX_CODE_VERSION?: string;
    readonly VX_SCREEN_ORIENTATION?: 'portrait' | 'landscape';
    readonly CACVOTE_MARK_WORKSPACE?: string;
    readonly CACVOTE_URL?: string;
    readonly VX_CA_CERT?: string;
    readonly CAC_ROOT_CA_CERTS?: string;
    readonly MACHINE_CERT?: string;
    readonly SIGNER?: string;
    readonly MAILING_LABEL_PRINTER?: string;
    readonly LIBNPRINT_WRAPPER_PATH?: string;
  }
}

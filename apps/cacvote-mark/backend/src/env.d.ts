declare namespace NodeJS {
  export interface ProcessEnv {
    readonly CI?: string;
    readonly NODE_ENV: 'development' | 'production' | 'test';
    readonly IS_INTEGRATION_TEST?: string;
    readonly USE_MOCK_CACVOTE_SERVER?: string;
    readonly BASE_PORT?: string;
    readonly PORT?: string;
    readonly VX_CODE_VERSION?: string;
    readonly VX_SCREEN_ORIENTATION?: 'portrait' | 'landscape';
    readonly CACVOTE_MARK_WORKSPACE?: string;
    readonly CACVOTE_URL?: string;
    readonly MAILING_LABEL_PRINTER?: string;
  }
}

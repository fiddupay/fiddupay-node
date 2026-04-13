/**
 * Safely extracts a displayable error message from various error formats.
 * Prevents React Error #31 by ensuring result is always a string.
 */
export const extractErrorMessage = (error: any, fallback = 'An unexpected error occurred'): string => {
    if (!error) return fallback;

    // 1. Check for string errors
    if (typeof error === 'string') return error;

    // 2. Check for backend-specific error objects from Axios/API
    // Current FidduPay structure: { error: { message: "xxx", code: "xxx" } } OR { error: "message" }
    const apiError = error.response?.data?.error;
    if (apiError) {
        if (typeof apiError === 'string') return apiError;
        if (typeof apiError === 'object') {
            return apiError.message || apiError.code || JSON.stringify(apiError);
        }
    }

    // 3. Fallback to standard JS error fields
    if (error.message) return error.message;

    // 4. Ultimate fallback: Stringify the object
    try {
        return typeof error === 'object' ? JSON.stringify(error) : String(error);
    } catch {
        return fallback;
    }
};

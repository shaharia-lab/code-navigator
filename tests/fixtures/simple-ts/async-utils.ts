// Async utilities

/**
 * Fetches user data
 */
export async function fetchUser(id: number): Promise<any> {
  const data = await fetchData(`/users/${id}`);
  return processUserData(data);
}

/**
 * Fetches data from API
 */
async function fetchData(url: string): Promise<any> {
  // Simulate API call
  return new Promise((resolve) => {
    setTimeout(() => resolve({ data: "mock" }), 100);
  });
}

/**
 * Processes user data
 */
function processUserData(data: any): any {
  return {
    ...data,
    processed: true,
  };
}

/**
 * Arrow function for validation
 */
const validateUser = async (user: any): Promise<boolean> => {
  const isValid = await checkValidation(user);
  return isValid;
};

/**
 * Checks validation
 */
async function checkValidation(user: any): Promise<boolean> {
  return user !== null && user !== undefined;
}

/**
 * Main function
 */
export async function main() {
  const user = await fetchUser(1);
  const isValid = await validateUser(user);

  if (isValid) {
    console.log("User is valid:", user);
  }
}
